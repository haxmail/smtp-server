package models

import "github.com/google/uuid"

type Email struct {
	ID        uuid.UUID `gorm:"primaryKey;type:uuid;default:uuid_generate_v4()" json:"id"`
	CreatedAt int       `json:"created_at"`
	UpdatedAt int       `json:"updated_at"`
	From      string    `json:"from"`
	To        string    `json:"to"`
	Subject   string    `json:"subject"`
	Body      string    `json:"body"`
}
