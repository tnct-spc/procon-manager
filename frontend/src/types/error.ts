export interface ApiError {
  response?: {
    status?: number
    data?: {
      message?: string
    }
  }
  message?: string
}

export const getErrorMessage = (error: unknown): string => {
  const apiError = error as ApiError
  return apiError.response?.data?.message || '操作に失敗しました'
}
