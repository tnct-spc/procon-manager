import axios from 'axios'

const getBaseURL = () => {
  const baseURL = import.meta.env.VITE_API_BASE_URL
  if (!baseURL) {
    throw new Error('VITE_API_BASE_URL is not defined in environment variables')
  }
  return baseURL
}

const api = axios.create({
  baseURL: `${getBaseURL()}/api/v1`,
  headers: { 'Content-Type': 'application/json' },
})

const authApi = axios.create({
  baseURL: getBaseURL(),
  headers: { 'Content-Type': 'application/json' },
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken')
  if (token && config.headers) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

export default api
export { getBaseURL, authApi }
