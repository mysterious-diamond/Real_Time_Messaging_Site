import { useEffect, useRef, useState } from "react";
import { getMessages } from "../api";
import { useParams } from "react-router-dom";

function ChatRoom() {
  const [error, setError] = useState("");
  const [messages, setMessages] = useState([]);
  const [cur_message, setCurrentMessage] = useState("");

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

  useEffect(() => {
    const token = localStorage.getItem("token");
    webSocket.current = new WebSocket(
      `${import.meta.env.VITE_WS_URL}/ws/${id}?token=${token}`,
    );

    webSocket.current.onmessage = (event) => {
      console.log(event.data);
      const msg = JSON.parse(event.data);
      setMessages((prev) => [...prev, msg]);
    };

    fetchMessages();

    return () => {
      webSocket.current.close();
    };
  }, []);

  return (
    <div className="chat-container">
      {error && <p className="error">{error}</p>}
      <div className="messages">
        {messages.map((msg, index) => (
          <div key={index} className="message">
            <strong>{msg.username}</strong>: {msg.content}
          </div>
        ))}
      </div>
      <div className="chat-input">
        <input
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
