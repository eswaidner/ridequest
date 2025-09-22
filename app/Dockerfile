FROM node:24.8-alpine

WORKDIR /app

COPY . ${WORKDIR}

RUN npm install

CMD ["npm", "run", "dev", "--", "--host"]