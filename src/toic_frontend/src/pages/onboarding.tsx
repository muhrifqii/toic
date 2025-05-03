import { onboardingSchema, OnboardingValues } from '@/lib/validations/onboarding'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { toic_backend } from '@declarations/toic_backend'
import { toast } from 'sonner'
import { useNavigate } from 'react-router'
import { mapToCategory, optionOf } from '@/lib/mapper'
import { CategoryName, categoryNames } from '@/types/core'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { Button, LoadingButton } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'

export default function OnboardingPage() {
  const navigate = useNavigate()
  const form = useForm<OnboardingValues>({
    resolver: zodResolver(onboardingSchema),
    defaultValues: { name: '', bio: '', categories: [] },
    mode: 'onChange'
  })

  const onSubmit = async ({ name, bio, categories }: OnboardingValues) => {
    try {
      toic_backend.complete_onboarding({
        name: optionOf(name),
        bio: optionOf(bio),
        categories: categories.map(mapToCategory)
      })
      toast.success("Welcome! You've received a limited time gifts 🥳")
      navigate('/')
    } catch (err) {
      toast.error('Failed to complete onboarding')
      console.error(err)
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className='max-w-xl mx-auto space-y-4 mt-10'>
        <h1 className='text-3xl font-bold'>Complete Your Profile</h1>

        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <FormItem>
              <FormLabel htmlFor='name'>Your Name</FormLabel>
              <FormControl>
                <Input placeholder='e.g. John Doe' {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='bio'
          render={({ field }) => (
            <FormItem>
              <FormLabel htmlFor='bio'>Your bio (optional)</FormLabel>
              <FormControl>
                <Textarea placeholder='A little bit about me' {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='categories'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Pick 3 categories you like</FormLabel>
              <FormControl>
                <div className='grid grid-cols-2 md:grid-cols-3 gap-3'>
                  {categoryNames.map(category => {
                    const isSelected = field.value.includes(category)
                    return (
                      <button
                        type='button'
                        key={category}
                        onClick={() => {
                          const next = isSelected
                            ? field.value.filter((c: string) => c !== category)
                            : [...field.value, category]
                          field.onChange(next)
                        }}
                        className={`rounded-lg border p-3 text-sm font-medium transition-all ${
                          isSelected
                            ? 'bg-primary text-white border-primary'
                            : 'bg-muted hover:bg-muted/80 border-muted-foreground'
                        }`}
                      >
                        {category}
                      </button>
                    )
                  })}
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <LoadingButton
          type='submit'
          isLoading={form.formState.isSubmitting}
          disabled={!form.formState.isValid}
          className='w-full'
        >
          Continue
        </LoadingButton>
      </form>
    </Form>
  )
}
