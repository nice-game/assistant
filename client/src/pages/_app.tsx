import { apollo$ } from "@/services/apollo";
import { ApolloProvider } from "@apollo/client";
import { AppProps } from "next/app";
import React from "react";
import { useObservable } from "rxjs-hooks";
import "../styles.scss";

export default function MyApp({ Component, pageProps }: AppProps) {
	const apollo = useObservable(() => apollo$);

	return (
		apollo && (
			<ApolloProvider client={apollo}>
				<Component {...pageProps} />
			</ApolloProvider>
		)
	);
}
