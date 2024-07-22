import { useParams } from "@solidjs/router"

export const Thread = () => {
    const params = useParams();
  
  return (
    <div class="thread-page">
      <h2>Thread view {params.id}</h2>
    </div>
  )
}