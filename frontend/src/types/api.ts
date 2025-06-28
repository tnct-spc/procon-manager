export interface User {
  id: string;
  name: string;
  email: string;
  role: "Admin" | "User";
}

export interface ItemCheckout {
  id: string;
  checkedOutBy: {
    id: string;
    name: string;
  };
  checkedOutAt: string;
}

export interface BaseItem {
  id: string;
  name: string;
  description: string;
  checkout?: ItemCheckout | null;
}

export interface GeneralItem extends BaseItem {
  category: "general";
}

export interface Book extends BaseItem {
  category: "book";
  author: string;
  isbn: string;
}

export interface Laptop extends BaseItem {
  category: "laptop";
  macAddress: string;
}

export type Item = GeneralItem | Book | Laptop;

export interface PaginatedItemResponse {
  total: number;
  limit: number;
  offset: number;
  items: Item[];
}

export interface CreateGeneralItemRequest {
  category: "general";
  name: string;
  description: string;
}

export interface CreateBookRequest {
  category: "book";
  name: string;
  author: string;
  isbn: string;
  description: string;
}

export interface CreateLaptopRequest {
  category: "laptop";
  name: string;
  mac_address: string;
  description: string;
}

export type CreateItemRequest =
  | CreateGeneralItemRequest
  | CreateBookRequest
  | CreateLaptopRequest;

export interface CheckoutResponse {
  id: string;
  checkedOutBy: string;
  checkedOutAt: string;
  returnedAt?: string;
  itemId: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  accessToken: string;
  userId: string;
}
