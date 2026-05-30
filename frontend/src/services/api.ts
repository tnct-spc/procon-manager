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
  credentials: 'include',
})

export default client
export { getBaseURL }
