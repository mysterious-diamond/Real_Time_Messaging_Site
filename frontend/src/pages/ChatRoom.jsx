import { useEffect, useRef, useState } from "react";
import { getMessages, getRoom, inviteUser } from "../api";
import { useParams, useNavigate } from "react-router-dom";

function ChatRoom() {
  const token = localStorage.getItem("token");
  const payload = JSON.parse(atob(token.split(".")[1]));
  const currentUserId = payload.sub;

  const [messages, setMessages] = useState([]);
  const [error, setError] = useState("");
  const [cur_message, setCurrentMessage] = useState("");
  const [inviteUsername, setInviteUsername] = useState("");
  const [initialLoad, setInitialLoad] = useState(true);
  const [room, setRoom] = useState(null);

  const { id } = useParams();
  const webSocket = useRef(null);
  const messagesEndRef = useRef(null);
  const messagesContainerRef = useRef(null);
  const navigate = useNavigate();

  const fetchMessages = async (scroll = false) => {
    try {
      const response = await getMessages(id);
      setMessages(response.data);
    } catch (err) {
      setError("Internal server error");
    }
  };

  const handleSend = () => {
    if (cur_message.trim() === "") return;
    webSocket.current.send(
      JSON.stringify({ type: "message", content: cur_message }),
    );
    setCurrentMessage("");
    scrollToBottom(true);
  };

  const handleDelete = async (messageId) => {
    try {
      webSocket.current.send(
        JSON.stringify({ type: "delete", message_id: messageId }),
      );
    } catch (err) {
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

  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  const formatDay = (timestamp) => {
    if (!timestamp) return null;
    const date = new Date(timestamp);
    const today = new Date();
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);

    if (date.toDateString() === today.toDateString()) return "Today";
    if (date.toDateString() === yesterday.toDateString()) return "Yesterday";
    return date.toLocaleDateString();
  };

  const shouldShowDay = (messages, index) => {
    if (index === 0) return true;
    const prev = new Date(messages[index - 1].created_at);
    const curr = new Date(messages[index].created_at);
    return prev.toDateString() !== curr.toDateString();
  };

  const isAtBottom = () => {
    const container = messagesContainerRef.current;
    if (!container) return true;
    return (
      container.scrollHeight - container.scrollTop - container.clientHeight < 50
    );
  };

  const scrollToBottom = (smooth = true) => {
    setTimeout(() => {
      if (messagesEndRef.current) {
        messagesEndRef.current.scrollIntoView({
          behavior: smooth ? "smooth" : "instant",
        });
      }
    }, 200);
  };

  useEffect(() => {
    webSocket.current = new WebSocket(
      `${import.meta.env.VITE_WS_URL}/ws/${id}?token=${token}`,
    );

    webSocket.current.onmessage = (event) => {
      const atBottom = isAtBottom();
      const msg = JSON.parse(event.data);

      if (msg.type === "delete") {
        setMessages((prev) =>
          prev.map((m) => (m.id === msg.message_id ? { ...m, deleted: 1 } : m)),
        );
      } else {
        setMessages((prev) => {
          if (prev.some((m) => m.id === msg.id)) return prev;
          return [...prev, msg];
        });
        if (atBottom) scrollToBottom(true);
      }
    };

    fetchMessages();
    fetchRoom();

    return () => {
      webSocket.current.close();
      setMessages([]);
    };
  }, []);

  // this one runs whenever messages is changed
  useEffect(() => {
    if (messages.length > 0 && initialLoad) {
      scrollToBottom(false); // instant on page load
      setInitialLoad(false);
    }
  }, [messages]);

  return (
    <div className="chat-container">
      <div className="chat-header">
        <button onClick={() => navigate("/lobby")}>← Back</button>
        <h2>{room ? room.name : "Loading..."}</h2>
      </div>
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

      <div className="messages" ref={messagesContainerRef}>
        {messages.map((msg, index) => (
          <div key={index}>
            {shouldShowDay(messages, index) && msg.created_at && (
              <div className="day-separator">{formatDay(msg.created_at)}</div>
            )}
            <div className="message">
              <strong>{msg.username}</strong>:{" "}
              {msg.deleted ? "message deleted" : msg.content}
              {msg.created_at && (
                <span className="message-time">
                  {formatTime(msg.created_at)}
                </span>
              )}
              {Number(msg.user_id) === Number(currentUserId) &&
                !msg.deleted && (
                  <button onClick={() => handleDelete(msg.id)}>Delete</button>
                )}
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input">
        <textarea
          value={cur_message}
          onChange={(e) => setCurrentMessage(e.target.value)}
          onKeyDown={(e) => {
            if (
              e.key === "Enter" &&
              !e.shiftKey &&
              !("ontouchstart" in window)
            ) {
              e.preventDefault();
              handleSend();
            }
          }}
          placeholder="Type a message..."
        />
        <button onClick={handleSend}>Send</button>
      </div>
    </div>
  );
}

export default ChatRoom;
