import { MDCTextField } from "@material/textfield";
import classNames from "classnames";

export interface TextfieldProps {
	className?: string;
	disabled?: boolean;
	inputRef?: React.LegacyRef<HTMLInputElement>;
	label?: string;
	onKeyPress?: React.KeyboardEventHandler<HTMLInputElement>;
	onKeyUp?: React.KeyboardEventHandler<HTMLInputElement>;
	outlined?: boolean;
	placeholder?: string;
	value?: string | number | readonly string[];
	valueChange?: (event: string) => void;
}
export const Textfield: React.FunctionComponent<TextfieldProps> = (props) => {
	const ref = (el: HTMLLabelElement) => el && new MDCTextField(el);

	const className = classNames(
		"mdc-text-field",
		props.outlined ? "mdc-text-field--outlined" : "mdc-text-field--filled",
		{ "mdc-text-field--no-label": !props.label, "mdc-text-field--disabled": props.disabled },
		props.className
	);

	const onChange = (e) => props.valueChange && props.valueChange(e.target.value);

	const label = <span className="mdc-floating-label">{props.label}</span>;

	return (
		<label ref={ref} className={className}>
			{props.outlined ? (
				<span className="mdc-notched-outline">
					<span className="mdc-notched-outline__leading"></span>
					{props.label && <span className="mdc-notched-outline__notch">{label}</span>}
					<span className="mdc-notched-outline__trailing"></span>
				</span>
			) : (
				<>
					<span className="mdc-text-field__ripple"></span>
					{label}
				</>
			)}
			<input
				ref={props.inputRef}
				className="mdc-text-field__input"
				placeholder={props.placeholder}
				disabled={props.disabled}
				onKeyPress={props.onKeyPress}
				onKeyUp={props.onKeyUp}
				value={props.value}
				onChange={onChange}
			/>
			{!props.outlined && <span className="mdc-line-ripple"></span>}
		</label>
	);
};
