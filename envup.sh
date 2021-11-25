# The following snippet exports (ie. makes availiable) the environment variables in the ".env" file.
if [ -f .env ]
then
  export $(cat .env | sed 's/#.*//g' | xargs)
fi

# This works with the script, but won't be availiable in docker-compose
export DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:${WL_POSTGRES_PORT}/${POSTGRES_DB}