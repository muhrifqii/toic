import { Outlet } from 'react-router'
import { Navbar } from '../blocks/navbar'

export default function MainLayout() {
  return (
    <>
      <Navbar />
      <Outlet />
    </>
  )
}
