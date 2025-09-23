import css from "./App.module.css";

const ENV = import.meta.env;

const authQueryParams = [
  `client_id=${ENV.VITE_STRAVA_CLIENT_ID}`,
  "redirect_uri=http://localhost:5173",
  "response_type=code",
  "approval_prompt=auto",
  "scope=activity:read",
].join("&");

const authUrl = `https://www.strava.com/oauth/authorize?${authQueryParams}`;

export default function App() {
  return (
    <div>
      <a href={authUrl} className={css.signIn}>
        Log in with Strava
      </a>
    </div>
  );
}
