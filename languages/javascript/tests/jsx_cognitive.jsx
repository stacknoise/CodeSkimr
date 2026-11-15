export function Form(props) {
  return <button onClick={() => { if (props.ok) { props.save(); } }}>save</button>;
}

export function Plain(props) {
  return <b onClick={props.save}>{props.ok ? "yes" : "no"}</b>;
}
