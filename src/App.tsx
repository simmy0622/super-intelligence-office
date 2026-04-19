import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "./components/AppShell";
import { Home } from "./pages/Home";
import { Notifications } from "./pages/Notifications";
import { Office } from "./pages/Office";
import { PostDetail } from "./pages/PostDetail";
import { Profile } from "./pages/Profile";
import { Search } from "./pages/Search";
import { Settings } from "./pages/Settings";
import { LanguageProvider } from "./lib/language";
import { SalonProvider } from "./lib/salon-context";

function App() {
  return (
    <BrowserRouter>
      <LanguageProvider>
        <SalonProvider>
          <Routes>
            <Route path="/office" element={<Office />} />
            <Route element={<AppShell />}>
              <Route index element={<Home />} />
              <Route path="/search" element={<Search />} />
              <Route path="/notifications" element={<Notifications />} />
              <Route path="/post/:id" element={<PostDetail />} />
              <Route path="/profile/:handle" element={<Profile />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Route>
          </Routes>
        </SalonProvider>
      </LanguageProvider>
    </BrowserRouter>
  );
}

export default App;
