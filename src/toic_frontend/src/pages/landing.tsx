import { Button } from '@/components/ui/button'
import { authService } from '@/services/auth'
import { useAuthStore } from '@/store/auth'
import { toast } from 'sonner'
import LoadingPage from './loading'
import { useNavigate } from 'react-router'
import { HeartHandshake, ListFilter, PenBox, PlusIcon } from 'lucide-react'
import HomeLayout from '@/components/layouts/home'

export default function LandingPage() {
  const navigate = useNavigate()
  const login = useAuthStore(state => state.login)
  const isAuthed = useAuthStore(state => state.isAuthenticated)
  const isHydrated = useAuthStore(state => state.isHydrated)

  const onLoginClicked = async () => {
    try {
      await login()
      toast.success('Login Successfull')
    } catch (reason: any) {
      toast.error(reason)
    }
  }

  if (!isHydrated) {
    return <LoadingPage />
  }

  if (!isAuthed) {
    return (
      <main className='w-full'>
        <section className='flex bg-[#E8A233] items-center justify-center'>
          <img src='/toic_full.png' className='my-20 w-3xl' />
        </section>

        <section className='text-center py-20 px-4 bg-gradient-to-b from-background to-muted/80'>
          <h1 className='text-4xl font-bold mb-4'>Write. Read. Inspire.</h1>
          <p className='text-lg text-muted-foreground mb-6 max-w-xl mx-auto'>
            Discover compelling stories, share your voice, and earn rewards — all on the decentralized web.
          </p>
          <Button size='lg' onClick={onLoginClicked}>
            Internet Identity Login
          </Button>
        </section>

        {/* How it Works */}
        <section className='py-16 px-4 max-w-5xl mx-auto text-center'>
          <h2 className='text-2xl font-semibold mb-8'>How it Works</h2>
          <div className='grid grid-cols-1 sm:grid-cols-3 gap-6'>
            <div className='bg-card shadow rounded-xl p-6'>
              <h3 className='text-xl font-bold mb-2'>Write</h3>
              <p className='text-muted-foreground'>
                Use our intuitive editor to draft stories that matter — fiction, essays, memoirs, and more.
              </p>
            </div>
            <div className='bg-card shadow rounded-xl p-6'>
              <h3 className='text-xl font-bold mb-2'>Publish</h3>
              <p className='text-muted-foreground'>
                Publish to the blockchain-backed platform. Your work stays yours, forever.
              </p>
            </div>
            <div className='bg-card shadow rounded-xl p-6'>
              <h3 className='text-xl font-bold mb-2'>Earn</h3>
              <p className='text-muted-foreground'>
                Receive token-based rewards as readers engage with your content — no ads, no middlemen.
              </p>
            </div>
          </div>
        </section>

        {/* Feature Highlights */}
        <section className='py-16 px-4 bg-gradient-to-b from-background to-muted/80'>
          <div className='max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-3 gap-8'>
            {[
              {
                title: 'Powerful Editor',
                desc: 'Craft beautiful stories with our rich editor. Markdown supported.',
                icon: <PenBox />
              },
              { title: 'Token Rewards', desc: 'Get rewarded for your creativity on-chain.', icon: <HeartHandshake /> },
              { title: 'Smart Feed', desc: 'Tailored stories based on your interest.', icon: <ListFilter /> }
            ].map(({ title, desc, icon }) => (
              <div key={title} className='bg-background p-6 rounded-xl shadow'>
                <span className='m-1'>{icon}</span>
                <h4 className='text-lg font-semibold mb-2'>{title}</h4>
                <p className='text-muted-foreground'>{desc}</p>
              </div>
            ))}
          </div>
        </section>

        {/* CTA */}
        <section className='py-20 text-center bg-background'>
          <h2 className='text-3xl font-bold mb-4'>Join the creative revolution</h2>
          <p className='text-muted-foreground mb-6'>Start writing and grows the community today — it's free.</p>
          <Button size='lg' onClick={onLoginClicked}>
            Get Started
          </Button>
        </section>
      </main>
    )
  }

  return (
    <>
      <main className='container mx-auto p-8 space-y-6'>
        <HomeLayout />
      </main>
    </>
  )
}
