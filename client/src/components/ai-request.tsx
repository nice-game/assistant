import React from "react";

interface Props {
	aiRequest: { query: string; reply?: string };
}
export const AiRequest: React.FC<Props> = (props) => {
	const { aiRequest } = props;

	return (
		<>
			<blockquote>{aiRequest.query}</blockquote>
			{aiRequest.reply ? <div>{aiRequest.reply}</div> : <div className="text-muted">Waiting for reply...</div>}
		</>
	);
};
