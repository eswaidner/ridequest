import { useEffect } from "react";
import css from "./App.module.css";

const ENV = import.meta.env;
const API_URL = "http://localhost:5174/api/v1";

const authQueryParams = [
  `client_id=${ENV.VITE_STRAVA_CLIENT_ID}`,
  "redirect_uri=http://localhost:5173",
  "response_type=code",
  "approval_prompt=auto",
  "scope=activity:read",
].join("&");

const authUrl = `https://www.strava.com/oauth/authorize?${authQueryParams}`;

export default function App() {
  useEffect(() => {
    const init = async () => {
      const resp = await fetch(`${API_URL}/auth/login`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ auth_code: "test" }),
      });

      console.log(await resp.json());
    };
    void init();
  }, []);

  return (
    <div>
      <a href={authUrl} className={css.signIn}>
        Log in with Strava
      </a>
    </div>
  );
}
