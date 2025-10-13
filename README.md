# therapia

🏥 A doctor appointment mobile application.

> [!IMPORTANT]
> This project is developed under the **01219493 Selected Topics in
Computer System Engineering** *(Software Architecture)* course of
**Department of Computer Engineering**, **Faculity of Engineering**,
**Kasetsart University**.

> [!WARNING]
> This project is still under development. Some features will be
> missing.

## Setup

TODO: add frontend setup instructions

### Installation

Clone this repository, copy `.env.example` to `.env` then fill out all the missing
enrironment variables.

#### Nix

Install [`devenv`](https://devenv.sh/getting-started/) then enter the
development shell either automatically via
[`nix-direnv`](https://github.com/nix-community/nix-direnv) or manually via running
`devenv shell` in the repository root, which will install all required dependencies.

Then, run `devenv up -d` to start all the required services in the background.

#### Manual

Install [PostgreSQL](https://www.postgresql.org/download/),
[Rust](https://rust-lang.org/tools/install/) and
[SQLx](https://github.com/launchbadge/sqlx).

Then, start the PostgreSQL database server.

---

Finally, run `cargo sqlx migrate run` to run initial database migrations.

### Usage

Run the backend via `cargo run` or `cargo run --release`. The API can now be accessed via
`http:$BIND_ADDR/api`, and their documentation can be accessed via
`http:$BIND_ADDR/docs/<visualiser>`. There are 4 visualisers available to use:
> [!NOTE]
> The trailing forward slash (`/`) in the URL for Swagger UI is required.
- [Swagger UI](https://swagger.io/tools/swagger-ui/): `.../swagger/`
- [RapiDoc](https://rapidocweb.com/): `.../rapidoc`
- [Redoc](https://redocly.github.io/redoc/): `.../redoc`
- [Scalar](https://scalar.com/): `.../scalar`

## Contributions

1. `6410500301` *ภูบดี สุตันรักษ์*
([@ItsZodiaX](https://github.com/ItsZodiaX)) - **Frontend Development**
2. `6610501955` *กฤชณัท ธนพิพัฒนศิริ*
([@krtchnt](https://github.com/krtchnt)) - **Backend Development**
3. `6610505276` *ก้องสกุล พันธุ์ยาง*
([@balliolon2](https://github.com/balliolon2)) - **Frontend Development**
4. `6610505560` *วรุตม์ มาศสุวรรณ*
([@nightyneko](https://github.com/nightyneko)) - **API & Database
Design**

## System Architecture

Service-based modular monolith, each service being layered

## Demo

TODO

## Acknowledgements

- [axum](https://docs.rs/axum/latest/axum/) - Ergonomic and modular
web framework built with Tokio, Tower, and Hyper
- [flutter](https://flutter.dev/) - Build apps for any screen
- ... TODO
