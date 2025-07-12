import createClient from 'openapi-fetch'
import type { paths } from '../types/schema'

const getBaseURL = () => {
  const baseURL = import.meta.env.VITE_API_BASE_URL
  if (!baseURL) {
    throw new Error('VITE_API_BASE_URL is not defined in environment variables')
  }
  return baseURL
}

// Create the main API client with authentication
const client = createClient<paths>({
  baseUrl: getBaseURL(),
})

// Add JWT authentication middleware
client.use({
  onRequest({ request }) {
    const token = localStorage.getItem('accessToken')
    if (token) {
      request.headers.set('Authorization', `Bearer ${token}`)
    }
    return request
  },
})

export default client
export { getBaseURL }
