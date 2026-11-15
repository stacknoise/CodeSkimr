type Props = { ok: boolean; items: number[]; save: () => void };

export function Form(props: Props) {
  return <button onClick={() => { if (props.ok) { props.save(); } }}>save</button>;
}

export function Plain(props: Props) {
  return <b onClick={props.save}>{props.ok ? "yes" : "no"}</b>;
}

export function Child(props: Props) {
  return <p>{() => { if (props.ok) { props.save(); } }}</p>;
}

export function Cast(props: Props) {
  run(function () { if (props.ok) { props.save(); } } as Handler);
  run((() => { if (props.ok) { props.save(); } }) as Handler);
  run((() => { if (props.ok) { props.save(); } })!);
  return null;
}
