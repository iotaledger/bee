---
description: Bee node usage examples.
image: /img/logo/bee_logo.png
keywords:
- examples
---
# Examples

## Generate a JWT for the REST API for your node
To generate a JWT for the REST API for your node run following command: 
`cargo release -- jwt-api`

Example auth for protected `/api/v1/tips` endpoint: 
```
curl -X 'GET' \
  'http://127.0.0.1:14265/api/v1/tips' \
  -H 'accept: application/json' \
  -H 'authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiIxMkQzS29vV1BydFg1cE1hYlZuaUJ5ZVo4OXh6OWVnWnhQcTRtNnNHNGV0V1Fwb2pjMkNmIiwic3ViIjoiQmVlIiwiYXVkIjoiYXBpIiwibmJmIjoxNjQ3Mjc2NDUxLCJpYXQiOjE2NDcyNzY0NTF9.WMprcjWTEVrszhfeoCEfg3-0nRRFrnRlUUtVeD78Xqs'
  ```

Note: you can protect desired REST API endpoints in the node configuration. Simply add the route you want to protect to the `protectedRoutes` array. To make routes public use the `publicRoutes` array.