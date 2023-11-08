# Avaruuskapakka v2.0

![6n4re7v3](https://github.com/emepi/kapchan-v2/assets/149962304/2e041a0a-febf-4e70-b889-e799f3cedded)

## Build

### Frontend

#### Install dependecies

- [Bun](https://bun.sh/) (or [Node.js](https://nodejs.org))

Next, install packages in */frontend* directory:

```bash
$ bun install # or npm install
```

#### Development

Run frontend from **development server**:

```bash
$ bun run dev # or npm run dev
```

Development server automatically picks up changes made to the frontend and 
performs HMR for real time browser updates. <br>
Open [http://localhost:5173](http://localhost:5173) to view it in the browser.

#### Deployment

Kapchan is using [vite](https://vitejs.dev/guide/static-deploy.html) build 
tools for frontend optimization:

```bash
$ bun vite build # or npm vite build
```

The minified build of the frontend should now appear in */dist* directory 
and is ready to be served from backend.

### Backend [TODO]

#### dependencies

- [rust](https://www.rust-lang.org/learn/get-started) 

#### Setup MySQL server [TODO]

##### dependencies

- [MySQL](https://dev.mysql.com/downloads/mysql/)
- [libmysqlclient-dev](https://dev.mysql.com/downloads/c-api/)
