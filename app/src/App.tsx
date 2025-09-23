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
    let authCode: string;
    const queryParams = window.location.search.slice(1).split("&");
    for (const p of queryParams) {
      const elems = p.split("=");
      if (elems[0] == "code") authCode = elems[1];
    }

    const login = async () => {
      if (!authCode) return;

      //TODO check auth scope and redirect if invalid

      await fetch(`${API_URL}/auth/login`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ auth_code: authCode }),
      });

      //TEST
      await fetch(`${API_URL}/auth/logout`, {
        method: "POST",
        credentials: "include",
      });
    };
    void login();
  }, []);

  return (
    <div>
      <a href={authUrl} className={css.signIn}>
        Log in with Strava
      </a>
    </div>
  );
}
