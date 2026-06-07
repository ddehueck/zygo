# Database

We use a sqlite database to store persistent data.

## Migrations

We use the sqlx-migrate crate to manage migrations.

## Running Migrations

To run migrations, use the `sqlx migrate run` command.

## Code Implications

The database models are used to represent the data in the database.
So when we want to add a new table to the database, we add a new database model.
Or when we update the schema of a table, we update the database model.

The domain models are used to represent the data in the domain.
So when the db models change we need update the adapters to convert between the database models and the domain models.