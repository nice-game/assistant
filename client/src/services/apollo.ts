import { ApolloClient, ApolloLink, HttpLink, InMemoryCache, split } from "@apollo/client";
import { setContext } from "@apollo/client/link/context";
import { WebSocketLink } from "@apollo/client/link/ws";
import { getMainDefinition } from "@apollo/client/utilities";
import { map } from "rxjs/operators";
import { session$ } from "./session";

export const apollo$ = session$.pipe(
	map((sessionUuid) => {
		const authLink = setContext(async (_, { headers }) => ({
			headers: { ...headers, authorization: sessionUuid ? sessionUuid : "" },
		}));

		const httpLink = authLink.concat(new HttpLink({ uri: "/graphql" }));

		let link = httpLink as ApolloLink;

		if (typeof window !== "undefined") {
			const wsLink = new WebSocketLink({
				uri: `ws://${window.location.hostname}:8000/graphql`,
				options: { connectionParams: { sessionUuid } },
			});

			link = split(
				({ query }) => {
					const definition = getMainDefinition(query);
					return definition.kind === "OperationDefinition" && definition.operation === "subscription";
				},
				wsLink || link,
				link
			);
		}

		return new ApolloClient({ cache: new InMemoryCache({}), link });
	})
);
