import { createFileRoute, Link } from '@tanstack/react-router';

export const Route = createFileRoute('/')({
  component: Home,
});

function Home() {
  return (
    // <HomeLayout {...baseOptions()}>
    //   <div className="flex flex-col items-center justify-center text-center flex-1">
    //     <h1 className="font-medium text-xl mb-4">Zygo documentation</h1>
    //     <Link
    //       to="/docs/$"
    //       params={{
    //         _splat: 'python',
    //       }}
    //       className="px-3 py-2 rounded-lg bg-fd-primary text-fd-primary-foreground font-medium text-sm mx-auto"
    //     >
    //       Open Docs
    //     </Link>
    //   </div>
    // </HomeLayout>
    <div className='w-full p-10 flex justify-center items-start'>
      <div className='w-lg'>
        <header className='mb-5'>
          <h1 className="font-bold text-lg font-mono mb-1">ZYGO</h1>
        </header>

        <p className='text-sm'>
          A python-first workflow system for bioinformatics.
          <br />
          <br />
          With zygo, you can define workflows entirely in Python. Zygo will automatically parallelize jobs and cache results so you can focus on research.
          <br />
          <br />
          Built with modern data systems in mind, zygo is designed to run on top of any object store - not just your local filesystem.
        </p>

        <div className='mt-8'>
          <Link
            to='/docs/$'
            params={{ _splat: 'python' }}
            className='px-3 py-2 rounded-xs bg-fd-primary text-fd-primary-foreground font-normal text-xs mx-auto'
          >
            Read the Documentation
          </Link>
        </div>
      </div>
    </div>
  );
}
