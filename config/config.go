package config

import (
	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
)

type S3EndpointConfig struct {
	Name      string
	AccessKey string
	SecretKey string
	Url       string
	Ssl       bool
}

type S3BucketPathConfig struct {
	Endpoint string
	Bucket   string
	SubPath  string
}

type Config struct {
	Endpoints []S3EndpointConfig
	Data      S3BucketPathConfig
	Pieces    S3BucketPathConfig
}

func (e *S3EndpointConfig) ToMinioClient() (*minio.Client, error) {
	return minio.New(e.Url, &minio.Options{
		Creds:  credentials.NewStaticV4(e.AccessKey, e.SecretKey, ""),
		Secure: e.Ssl,
	})
}
