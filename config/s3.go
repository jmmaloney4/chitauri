package config

type S3EndpointConfig struct {
	Name      string
	AccessKey string
	SecretKey string
	Url       string
	Ssl       bool
}

type Config struct {
	Endpoints []S3EndpointConfig
}
