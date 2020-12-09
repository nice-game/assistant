import { AiRequest } from "@/components/ai-request";
import { Grid } from "@/components/material/layout-grid";
import { Textfield } from "@/components/material/textfield";
import { gql, useMutation, useSubscription } from "@apollo/client";
import Head from "next/head";
import React from "react";

const Home: React.FC = () => {
	const [msgs, setMsgs] = React.useState([] as AiRequest[]);
	const [query, setQuery] = React.useState("");
	const [aiRequestId, setAiRequestId] = React.useState<number>(null);

	const [createRequest, { loading: loadingCreate, error }] = useMutation(createRequestMutation, {
		onCompleted: (data) => setAiRequestId(data.createAiRequest.id),
	});
	useSubscription(onAiReplySubscription, {
		variables: { aiRequestId },
		skip: !aiRequestId,
		onSubscriptionData: (options) => {
			const data = options.subscriptionData.data;
			if (data) {
				msgs[msgs.length - 1].reply = data.aiReply.reply;
			}
			setMsgs([...msgs]);
		},
	});

	const onKeyPress = (event: React.KeyboardEvent<HTMLInputElement>) => {
		if (event.key === "Enter") {
			createRequest({ variables: { text: query } });
			setMsgs([...msgs, { query }]);
			setQuery("");
		}
	};

	return (
		<>
			<Head>
				<title>Assistant</title>
			</Head>

			<Grid className="container h-100vh p-3">
				<Grid.Cell span={12} className="h-100 d-flex flex-col">
					<div className="flex-grow-1 overflow-y-auto bg-gray4 p-3 mb-3">
						{msgs.map((msg, i) => (
							<AiRequest key={i} aiRequest={msg} />
						))}
					</div>
					<Textfield
						label="Enter message"
						value={query}
						valueChange={(x) => setQuery(x)}
						onKeyPress={onKeyPress}
						outlined
					/>
				</Grid.Cell>
			</Grid>
		</>
	);
};
export default Home;

interface AiRequest {
	query: string;
	reply?: string;
}

const createRequestMutation = gql`
	mutation CreateAiRequest($text: String!) {
		createAiRequest(text: $text) {
			id
		}
	}
`;

const onAiReplySubscription = gql`
	subscription AiReply($aiRequestId: Int!) {
		aiReply(aiRequestId: $aiRequestId) {
			reply
		}
	}
`;
