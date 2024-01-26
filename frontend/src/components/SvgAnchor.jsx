import { formatClass } from "../scripts/utils"

export const SvgAnchor = (props) => {
  return (
    <a class={"svg-a" + formatClass(props.class, true)} href={props.href}>
      <svg class={formatClass(props.iconClass)} xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24">
        <path fill="currentColor" d={props.icon}/>
      </svg>
      {props.label}
    </a>
  )
}