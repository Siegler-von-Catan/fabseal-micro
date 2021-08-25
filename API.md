# API

## User Feedback

* Endpoint `GET /api/v1/feedback/<dataset_name>/<image_id>`
  Content-Type: `application/json`

### Retrieving existing feedback

* Endpoint: `GET /api/v1/feedback/<dataset_name>/<image_id>/<feedback_id>/<feedback_image_id>`
  Content-Type: `image/jpeg` etc.

OR

* Endpoint: `GET /data/feedback/images/<feedback_image_id>.jpg`

### Posting new feedback

* Endpoint `POST /api/v1/feedback/<dataset_name>/<image_id>/new`
  `200 OK id=<request_id>`

* Endpoint `POST /api/v1/feedback/<dataset_name>/<image_id>/metadata?id=<request_id>`
  Content-Type: `application/json`

* Endpoint `POST /api/v1/feedback/<dataset_name>/<image_id>/upload?id=<request_id>`
  Content-Type: `image/jpeg`

* Endpoint `POST /api/v1/feedback/<dataset_name>/<image_id>/finish?id=<request_id>`
  `200 OK id=<feedback_id>`

## FabSeal Create

### Retrieving creations

* Endpoint: `GET /api/v1/userupload/public/result?id=<upload_id>&type=model`
  Content-Type: `model/stl` (or glTF, OBJ?)

* Endpoint: `GET /api/v1/userupload/public/result?id=<upload_id>&type=heightmap`
  Content-Type: `image/jpeg`?

### New

* Endpoint `POST /api/v1/create/new`
  `200 OK`
  (sets cookie)

* Endpoint `POST /api/v1/create/upload`
  Content-Type: `image/jpeg` or `image/png`

* Endpoint `POST /api/v1/create/start`
  Content-Type: `application/json` (constaining request settings)

* Endpoint `GET /api/v1/create/result?type=heightmap`
  Content-Type: `image/jpeg`? (not implemented)

* Endpoint `GET /api/v1/create/result?type=model`
  Content-Type: `model/stl`

* Endpoint `POST /api/v1/create/finish`
  `200 OK id=<upload_id>` (not implemented)


# Internal API

