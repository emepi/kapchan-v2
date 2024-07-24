import { formatClass } from "../scripts/utils"
import { session } from "../scripts/user_service"


export const RadioButton = (props) => (
  <Show 
    when={session() && session().role >= props.accessLevel}
    fallback={
      <label class={"radio-btn" + formatClass(props.class, true)}>
        <span class="radio-lck">{props.children}</span>
      </label>
    }>
    <label class={"radio-btn" + formatClass(props.class, true)}>
      <input class="radio-in" type="radio" onChange={props.onChange} name={props.name} value={props.value} checked={props.checked}/>
      <span class={"radio-lbl" + formatClass(props.labelClass, true)}>{props.children}</span>
    </label>
  </Show>
)