import React, { useCallback, useEffect, useState } from "react";
import Box from "@mui/material/Box";
import { TextDivider } from "@components/Divider/TextDivider";
import { Text } from "@components/Text/TextComponent";
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useProgramApis } from "../../providers/ProgramApisProvider";
import { myGetTokenMetadata } from "../../../utils/token2022";
import CircularProgress from '@mui/material/CircularProgress';
import { Carousel } from "./Carousel";
import { EpNFTContainer } from "./EpNFTContainer";
import { useSearchParams } from "next/navigation";
import { PublicKey } from "@solana/web3.js";
import {TokenMetadata} from  "@solana/spl-token-metadata";

export function MyEpNFTs() {
    const [isFetching, setIsFetching] = useState<boolean>(true);
    const [tokens, setTokens] = useState<TokenMetadata[]>([]);
    const {program, hasCreatedtState: {hasCreated}} = useProgramApis();
    const path = useSearchParams();

    const fetchNFTs = useCallback(async (program) => {
        setIsFetching(true);
        try {
            let pubkey: PublicKey;
            const urlPath = path.get("owner");
            if (urlPath !== null) {
                pubkey = new PublicKey(urlPath);
            } else if (program.wallet !== undefined) {
                pubkey = program.wallet.publicKey;
            } else {
                throw new Error("No publickey");
            }

            const tokens = await myGetTokenMetadata(program.connection, pubkey);
            setTokens(tokens);
        } catch (e) {
            console.log("Failed getting NFTs", e);
        } finally {
            setIsFetching(false);
        }
    }, [hasCreated]);

    useEffect(() => {
        fetchNFTs(program).then();
    }, [program, fetchNFTs]);

    return (
        <Box
            component="div"
            position="relative"
            height={"100%"}
            rowGap={"16px"}
            display={"flex"}
            flexDirection={"column"}
            alignSelf={"start"}
            width={{ sm: "300px", md: "400px" }}
        >
            <div className={"w-full"}>
                <TextDivider>My epNFTs</TextDivider>
            </div>

            <div className="flex justify-center self-center items-center w-full flex-col">
                {isFetching ? <CircularProgress sx={{color: "secondary.main"}} /> :
                    <>
                        { tokens.length === 0 ?
                            <div className={"flex items-center gap-x-2"}>
                                <ArrowBackIcon sx={{color: "secondary.main"}}/>
                                <Text.H6>
                                    Create an ephemeral epNFT
                                </Text.H6>
                            </div>
                            : <Carousel
                                items={tokens}
                                ItemComponent={EpNFTContainer}
                            />
                        }
                    </>
                }
            </div>
        </Box>
    );
}
