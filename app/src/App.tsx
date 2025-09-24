import { useEffect, useState } from "react";
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
  const [loggedIn, setLoggedIn] = useState<boolean>(false);

  useEffect(() => {
    let authCode: string;
    const queryParams = window.location.search.slice(1).split("&");
    for (const p of queryParams) {
      const elems = p.split("=");
      if (elems[0] == "code") authCode = elems[1];
    }

    // reset the url display
    window.history.replaceState(null, "home", window.location.hostname);

    const login = async () => {
      if (!authCode) return;

      //TODO check auth scopes and redirect if invalid

      await fetch(`${API_URL}/auth/login`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ auth_code: authCode }),
      });

      setLoggedIn(true);
    };
    void login();
  }, []);

  console.log(loggedIn);

  return (
    <div>
      {loggedIn ? (
        <button
          className={css.logOut}
          onClick={async () => {
            await fetch(`${API_URL}/auth/logout`, {
              method: "POST",
              credentials: "include",
            });

            setLoggedIn(false);
          }}
        >
          Log Out
        </button>
      ) : (
        <a href={authUrl} className={css.logIn}>
          Log in with Strava
        </a>
      )}
    </div>
  );
}
