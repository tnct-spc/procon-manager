// Re-export commonly used types from schema.d.ts for convenience
import type { paths, components } from './schema'

export type { paths }

// Convenience type aliases using OpenAPI schema
export type User = components['schemas']['UserResponse']
export type Item = components['schemas']['ItemResponse']
export type CreateItemRequest = components['schemas']['CreateItemRequest']
export type CreateUserRequest = components['schemas']['CreateUserRequest']
export type UpdateItemRequest = components['schemas']['UpdateItemRequest']
export type LoginRequest = components['schemas']['LoginRequest']
export type LoginResponse = components['schemas']['AccessTokenResponse']
export type PaginatedItemResponse = components['schemas']['PaginatedItemResponse']
export type CheckoutResponse = components['schemas']['CheckoutResponse']
export type CheckoutsResponse = components['schemas']['CheckoutsResponse']
export type ItemCheckoutResponse = components['schemas']['ItemCheckoutResponse']
export type CheckoutUser = components['schemas']['CheckoutUser']
export type UpdateUserPasswordRequest = components['schemas']['UpdateUserPasswordRequest']
export type UpdateUserRoleRequest = components['schemas']['UpdateUserRoleRequest']
export type UpdateUserNameRequest = components['schemas']['UpdateUserNameRequest']
export type UpdateUserEmailRequest = components['schemas']['UpdateUserEmailRequest']
export type ItemCategory = components['schemas']['ItemCategory']
