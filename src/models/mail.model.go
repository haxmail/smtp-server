package models

import "gorm.io/gorm"

type Mail struct {
	gorm.Model
	From string
	To   string
	Data string
}
