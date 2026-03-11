import { useEffect, useRef, useState } from "react";
import { getMessages, deleteMessage, getRoom, inviteUser } from "../api";
import { useParams } from "react-router-dom";

function ChatRoom() {
  const token = localStorage.getItem("token");
  const payload = JSON.parse(atob(token.split(".")[1]));
  const currentUserId = payload.sub;

  const [messages, setMessages] = useState([]);
  const [error, setError] = useState("");
  const [cur_message, setCurrentMessage] = useState("");
  const [inviteUsername, setInviteUsername] = useState("");
  const [room, setRoom] = useState(null);

  const { id } = useParams();
  const webSocket = useRef(null);

  const fetchMessages = async () => {
    try {
      const response = await getMessages(id);
      setMessages(response.data);
    } catch (err) {
      setError("Internal server error");
    }
  };

  const handleSend = () => {
    webSocket.current.send(cur_message);
    setCurrentMessage("");
  };

  const handleDelete = async (messageId) => {
    console.log("Deleting message", messageId);
    try {
      const response = await deleteMessage(messageId);
      console.log("Delete response", response);
      fetchMessages();
    } catch (err) {
      console.log("Delete error", err);
      setError("Failed to delete message");
    }
  };

  const fetchRoom = async () => {
    try {
      const response = await getRoom(id);
      setRoom(response.data);
    } catch (err) {
      setError("Failed to load room");
    }
  };

  const handleInvite = async () => {
    try {
      await inviteUser(id, inviteUsername);
      setInviteUsername("");
      alert("User invited!");
    } catch (err) {
      setError("Failed to invite user");
    }
  };

  useEffect(() => {
    webSocket.current = new WebSocket(
      `${import.meta.env.VITE_WS_URL}/ws/${id}?token=${token}`,
    );

    webSocket.current.onmessage = (event) => {
      console.log(event.data);
      const msg = JSON.parse(event.data);
      setMessages((prev) => [...prev, msg]);
    };

    fetchMessages();
    fetchRoom();

    return () => {
      webSocket.current.close();
    };
  }, []);

  return (
    <div className="chat-container">
      {error && <p className="error">{error}</p>}

      {room && room.created_by === currentUserId && room.is_private === 1 && (
        <div className="invite-form">
          <input
            type="text"
            placeholder="Invite username..."
            value={inviteUsername}
            onChange={(e) => setInviteUsername(e.target.value)}
          />
          <button onClick={handleInvite}>Invite</button>
        </div>
      )}

      <div className="messages">
        {messages.map((msg, index) => (
          <div key={index} className="message">
            <strong>{msg.username}</strong>:{" "}
            {msg.deleted ? "message deleted" : msg.content}
            {msg.user_id === currentUserId && !msg.deleted && (
              <button onClick={() => handleDelete(msg.id)}>Delete</button>
            )}
          </div>
        ))}
      </div>

      <div className="chat-input">
        <textarea
          value={cur_message}
          onChange={(e) => setCurrentMessage(e.target.value)}
          placeholder="Type a message..."
        />
        <button onClick={handleSend}>Send</button>
      </div>
    </div>
  );
}

export default ChatRoom;
