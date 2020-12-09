import { MDCRipple } from '@material/ripple';
import classNames from "classnames";

export interface ButtonProps {
	children?: React.ReactNode;
	className?: string;
	disabled?: boolean;
	type?: "text" | "outlined" | "raised";
	['aria-label']?: string;
}
export const Button: React.FunctionComponent<ButtonProps> = (props) => {
	const ref = (el: HTMLButtonElement) => el && new MDCRipple(el);

	const className = classNames("mdc-button", { "mdc-button--outlined": props.type === "outlined", "mdc-button--raised": props.type === "raised" }, props.className);

	return <button ref={ref} className={className} disabled={props.disabled}>
		<div className="mdc-button__ripple"></div>
		<span className="mdc-button__label">{props.children}</span>
	</button>
};
