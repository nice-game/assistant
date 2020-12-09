import { MDCRipple } from '@material/ripple';
import { mdiPlus } from '@mdi/js';
import Icon from '@mdi/react';

export interface FabProps {
	children?: React.ReactNode;
	className?: string;
	['aria-label']?: string;
}
export const Fab: React.FunctionComponent<FabProps> = (props) => {
	const ref = (el: HTMLButtonElement) => el && new MDCRipple(el);

	return <button ref={ref} className={`mdc-fab ${props.className}`} aria-label={props['aria-label']}>
		<div className="mdc-fab__ripple"></div>
		<Icon path={mdiPlus} className="mdc-fab__icon" />
		<style jsx>{`
			.mdc-fab {
				position: absolute;
				right: 3rem;
				bottom: 3rem;
			}
		`}</style>
	</button>
};
