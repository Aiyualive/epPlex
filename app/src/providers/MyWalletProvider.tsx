import React, { useCallback, useMemo } from "react";
import { WalletError } from "@solana/wallet-adapter-base";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import {
    PhantomWalletAdapter,
    SolflareWalletAdapter,
    BackpackWalletAdapter,
    SlopeWalletAdapter,
    GlowWalletAdapter,
    TorusWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import toast from "react-hot-toast";
import { CustomWalletDialogProvider } from "../components/Dialogs/MyWalletDialog/MyWalletDialogProvider";
// import useMyConnection from "./useMyConnection";
// import MyXnftContextProvider from "./MyXnftProvider";
import { COMMITMENT } from "../../client/constants";

const MyWalletProvider = ({ children }) => {
    // const currentEndpoint = undefined;

    const endpoint = useMemo(() => {
        // Probably this is unnecessary since it is already set within the default
        // if (currentEndpoint === undefined) {
        console.log("process.env.NEXT_PUBLIC_SOLANA_RPC_HOST", process.env.NEXT_PUBLIC_SOLANA_RPC_HOST);
        return process.env.NEXT_PUBLIC_SOLANA_RPC_HOST as string;
        // }

        // return currentEndpoint.url;
    }, []);

    // Not even necessary to use this
    // https://twitter.com/burger606/status/1649453569651736587?s=20
    const wallets = useMemo(
        () => [
            new PhantomWalletAdapter(),
            new SolflareWalletAdapter(),
            new BackpackWalletAdapter(),
            new GlowWalletAdapter(),
            new SlopeWalletAdapter(),
            new TorusWalletAdapter(),
        ],
        [endpoint]
    );

    const onError = useCallback((error: WalletError) => {
        toast.error(error.message ? `${error.name}: ${error.message}` : error.name);
    }, []);

    return (
        <ConnectionProvider endpoint={endpoint} config={{ commitment: COMMITMENT }}>
            <WalletProvider wallets={wallets} onError={onError} autoConnect={true}>
                <CustomWalletDialogProvider>
                    {children}
                </CustomWalletDialogProvider>
            </WalletProvider>
        </ConnectionProvider>
    );
};

export default MyWalletProvider;