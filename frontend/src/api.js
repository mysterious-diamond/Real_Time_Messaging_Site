import axios from 'axios'

const api = axios.create({
    baseURL: import.meta.env.VITE_API_URL
});

// Automatically attach JWT token to every request
api.interceptors.request.use((config) => {
    const token = localStorage.getItem('token')
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }
    return config
})

export const register = (username, password) =>
    api.post('/register', { username, password })

export const login = (username, password) =>
    api.post('/login', { username, password })

export const getRooms = () =>
    api.get('/rooms')

export const getMessages = (roomId) =>
    api.get(`/rooms/${roomId}/messages`)

export const deleteMessage = (messageId) =>
    api.delete(`/messages/${messageId}`)

export const createRoom = (name, isPrivate) =>
    api.post('/rooms', { name, is_private: isPrivate })

export const inviteUser = (roomId, username) =>
    api.post(`/rooms/${roomId}/invite`, { username })

export const getRoom = (roomId) =>
    api.get(`/rooms/${roomId}`)

export default api