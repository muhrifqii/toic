import { PropWithChild } from '@/types/ui'
import { useNavigate } from 'react-router-dom'

export default function AuthGuard({ children }: PropWithChild) {
  const nav = useNavigate()

  return children
}
