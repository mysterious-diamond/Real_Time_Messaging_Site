import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { login, register } from "../api";

function Login() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const navigate = useNavigate();

  const handleLogin = async () => {
    try {
      const response = await login(username, password);
      localStorage.setItem("token", response.data);
      navigate("/lobby");
    } catch (err) {
      setError("Invalid username or password");
    }
  };

  const handleRegister = async () => {
    try {
      await register(username, password);
      await handleLogin();
    } catch (err) {
      setError("Username already exists");
    }
  };

  return (
    <div className="page">
      <div className="card">
        <h1>Chat App</h1>
        {error && <p className="error">{error}</p>}
        <input
          type="text"
          placeholder="Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <button onClick={handleLogin}>Login</button>
        <button onClick={handleRegister}>Register</button>
      </div>
    </div>
  );
}

export default Login;
