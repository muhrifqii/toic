import { Outlet } from 'react-router'
import { NavbarEditor } from '../blocks/navbar-editor'

export default function EditorLayout() {
  return (
    <>
      <NavbarEditor />
      <Outlet />
    </>
  )
}
