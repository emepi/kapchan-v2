import { useParams } from "@solidjs/router";

export function Boards() {
  const params = useParams();

    return (
        <div>
          <section>
            <h2>{params.board}</h2>
          </section>
        </div>
    )
}