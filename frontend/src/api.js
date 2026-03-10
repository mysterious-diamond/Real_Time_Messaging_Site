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

export const createRoom = (name) =>
    api.post('/rooms', { name })

export const getMessages = (roomId) =>
    api.get(`/rooms/${roomId}/messages`)

export default api