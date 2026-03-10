import { useEffect, useState } from "react";
import { createRoom, getRooms } from "../api";
import { useNavigate } from "react-router-dom";

function Lobby() {
  const [rooms, setRooms] = useState([]);
  const [error, setError] = useState("");
  const [newRoom, setNewRoom] = useState("");
  const navigate = useNavigate();

  const fetchRooms = async () => {
    try {
      const response = await getRooms();
      setRooms(response.data);
    } catch (err) {
      setError("Internal server error");
    }
  };

  const handleCreateRoom = async () => {
    try {
      await createRoom(newRoom);
      setNewRoom("");
      fetchRooms();
    } catch (err) {
      setError("Internal server error");
    }
  };

  useEffect(() => {
    fetchRooms();
  }, []);

  return (
    <div className="page">
      <h1>Rooms</h1>
      {error && <p className="error">{error}</p>}
      <div className="room-list">
        {rooms.map((room) => (
          <div
            key={room.id}
            className="room-item"
            onClick={() => navigate(`/rooms/${room.id}`)}
          >
            {room.name}
          </div>
        ))}
      </div>
      <div className="create-room">
        <input
          type="text"
          placeholder="New room name"
          value={newRoom}
          onChange={(e) => setNewRoom(e.target.value)}
        />
        <button onClick={handleCreateRoom}>Create</button>
      </div>
    </div>
  );
}

export default Lobby;
