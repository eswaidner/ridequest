import { useEffect, useState, type ReactNode } from "react";
import css from "./App.module.css";

const ENV = import.meta.env;
const API_URL = "http://localhost:5174/api/v1";

// strava auth url query params
const authQueryParams = [
  `client_id=${ENV.VITE_STRAVA_CLIENT_ID}`,
  "redirect_uri=http://localhost:5173",
  "response_type=code",
  "approval_prompt=auto",
  "scope=activity:read",
].join("&");

const authUrl = `https://www.strava.com/oauth/authorize?${authQueryParams}`;

export default function App() {
  //TODO replace this with useLoginState query
  // locally track the login state
  const [loginState, setLoginState] = useState<
    "logged-in" | "pending" | "logged-out"
  >("logged-out");

  useEffect(() => {
    // extract auth code from url query params
    let authCode: string;
    const queryParams = window.location.search.slice(1).split("&");
    for (const p of queryParams) {
      const elems = p.split("=");
      if (elems[0] == "code") authCode = elems[1];
    }

    // reset the url display
    window.history.replaceState(null, "RideQuest", window.location.hostname);

    const login = async () => {
      if (!authCode) return;

      //TODO check auth scopes and show error if invalid

      // send authorization code to backend for token exchange
      await fetch(`${API_URL}/auth/login`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ auth_code: authCode }),
      });

      //TODO handle errors

      setLoginState("logged-in");
    };
    void login();
  }, []);

  let logInOutBtn: ReactNode;

  // select ui based on login state
  switch (loginState) {
    case "logged-in":
      logInOutBtn = (
        <LogOutButton
          onClick={() => {
            setLoginState("logged-out");
          }}
        />
      );
      break;

    case "logged-out":
      logInOutBtn = (
        <LogInButton
          onClick={() => {
            setLoginState("pending");
          }}
        />
      );
      break;

    case "pending":
      logInOutBtn = <>Loading...</>;
      break;
  }

  return (
    <div className={css.app}>
      <img
        src="./ridequest.png"
        alt="RideQuest logo"
        width={100}
        height={100}
      ></img>
      {logInOutBtn}
    </div>
  );
}

interface LogInButtonProps {
  onClick?: () => void;
}

function LogInButton({ onClick }: LogInButtonProps) {
  return (
    <a
      onClick={() => {
        onClick?.();
      }}
      href={authUrl}
      className={css.logInOutButton}
    >
      Log in with Strava
    </a>
  );
}

interface LogOutButtonProps {
  onClick?: () => void;
}

function LogOutButton({ onClick }: LogOutButtonProps) {
  return (
    <button
      className={css.logInOutButton}
      onClick={() => {
        fetch(`${API_URL}/auth/logout`, {
          method: "POST",
          credentials: "include",
        })
          .then(() => {
            onClick?.();
          })
          .catch(() => {
            console.log("ERROR: failed to log in");
          });
      }}
    >
      Log Out
    </button>
  );
}
