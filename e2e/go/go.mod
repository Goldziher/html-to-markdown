module github.com/xberg-io/html-to-markdown/packages/go/v3/e2e

go 1.26

require (
	github.com/stretchr/testify v1.12.1
	github.com/xberg-io/html-to-markdown/packages/go/v3 v3.18.1
)

require go.yaml.in/yaml/v3 v3.0.5 // indirect

replace github.com/xberg-io/html-to-markdown/packages/go/v3 => ../../packages/go
