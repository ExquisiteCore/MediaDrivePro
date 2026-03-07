import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './store/auth'
import { useEffect } from 'react'
import Layout from './components/Layout'
import ProtectedRoute from './components/ProtectedRoute'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'
import FileBrowserPage from './pages/FileBrowserPage'
import SharesPage from './pages/SharesPage'
import TokensPage from './pages/TokensPage'
import SettingsPage from './pages/SettingsPage'
import AdminPage from './pages/AdminPage'
import PublicSharePage from './pages/PublicSharePage'
import ImagesPage from './pages/ImagesPage'

export default function App() {
  const { token, loadUser } = useAuthStore()

  useEffect(() => {
    if (token) {
      loadUser()
    }
  }, [token, loadUser])

  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route path="/s/:token" element={<PublicSharePage />} />
      <Route element={<ProtectedRoute />}>
        <Route element={<Layout />}>
          <Route path="/" element={<Navigate to="/files" replace />} />
          <Route path="/files" element={<FileBrowserPage />} />
          <Route path="/files/folder/:folderId" element={<FileBrowserPage />} />
          <Route path="/shares" element={<SharesPage />} />
          <Route path="/images" element={<ImagesPage />} />
          <Route path="/tokens" element={<TokensPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/admin" element={<AdminPage />} />
        </Route>
      </Route>
    </Routes>
  )
}
