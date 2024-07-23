import { Show, createMemo, createResource, createSignal } from "solid-js"
import { useParams } from "@solidjs/router"
import { credentials } from "../scripts/credentials"
import { session, startSession } from "../scripts/user_service"

export const Thread = () => {
  const params = useParams();
  const [posts] = createResource(async () => (await fetch("/threads/" + params.id)).json())

  const post = async (e) => {
    e.preventDefault()
    const data = new FormData(e.target)

    console.log(data)

    if (!credentials.access_token) {
      await startSession()
    }

    if (credentials.access_token) {
      fetch("/threads/" + params.id, {
        method: "POST",
        headers: [["Authorization", "Bearer " + credentials.access_token]],
        body: data,
      })
    }
  }
  
  return (
    <Show when={!posts.loading}>
      <div class="thread-page">
        <div class="op-post">
          <Show when={posts().op_post.attachment}>
            <img class="post-img" src={"/files/" + posts().op_post.post_id}></img>
          </Show>
          <div>
            <p class="post-info">No. {posts().op_post.post_id} Created: {(new Date(posts().op_post.created_at)).toLocaleString("fi-FI")}</p>
            <h2>{posts().title}</h2>
            <p>{posts().op_post.body}</p>
          </div>
        </div>

        <For each={posts().responses}>
          { (post) => 
          <div class="post">
            <Show when={post.attachment}>
              <img class="post-img" src={"/files/" + post.post_id}></img>
            </Show>
            <div>
              <p class="post-info">No. {post.post_id} Created: {(new Date(post.created_at)).toLocaleString("fi-FI")}</p>
              <p>{post.body}</p>
            </div>
          </div> }
        </For>

        <div class="reply-box">
          <form class="reply-form" onSubmit={post}>
            <textarea class="post-body" name="body" placeholder="Reply..." rows="1" cols="150" maxLength="40000"/>
            <label>
                <input type="file" name="attachment" accept="image/png, image/jpeg" hidden/>
                <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="M720-330q0 104-73 177T470-80q-104 0-177-73t-73-177v-370q0-75 52.5-127.5T400-880q75 0 127.5 52.5T580-700v350q0 46-32 78t-78 32q-46 0-78-32t-32-78v-370h80v370q0 13 8.5 21.5T470-320q13 0 21.5-8.5T500-350v-350q-1-42-29.5-71T400-800q-42 0-71 29t-29 71v370q-1 71 49 120.5T470-160q70 0 119-49.5T640-330v-390h80v390Z"/></svg>
            </label>
            <button>
              <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24"><path fill="currentColor" d="M120-160v-640l760 320-760 320Zm80-120 474-200-474-200v140l240 60-240 60v140Zm0 0v-400 400Z"/></svg>
            </button>
          </form>
        </div>
      </div>
    </Show>
  )
}