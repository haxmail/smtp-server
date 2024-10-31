package config

import "github.com/spf13/viper"

var (
	DB_URI                  = ""
	MIGRATE            bool = false
	TELEGRAM_BOT_TOKEN      = ""
)

func LoadCfg() {
	DB_URI = viper.GetString("DB_URI")
	MIGRATE = viper.GetBool("MIGRATE")
}
