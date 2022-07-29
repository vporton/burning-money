import React from "react";
// import { ethers } from 'hardhat';
import { ethers } from 'ethers';
import Calendar from 'react-calendar'
import 'react-calendar/dist/Calendar.css';
import deployed from "./deployed-addresses.json";
import { CHAINS } from './data';
import Token from "./Token.json";
import { useEffect, useState } from 'react';
const { utils, BigNumber: BN } = ethers;
const tokenAbi = Token.abi;

export default function Withdraw() {
    const maxDate = new Date();
    // maxDate.setDate(maxDate.getDate() - 1);
    const [date, setDate] = useState(maxDate);
    const [amount, setAmount] = useState<number>(0);
    const [withdrawn, setWithdrawn] = useState(false);

    useEffect(() => {
        // TODO: Duplicate code
        (window as any).ethereum.enable().then(async () => {
            const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
            const { chainId } = await provider.getNetwork();
            const addrs = (deployed as any)[CHAINS[chainId]];
            const day = Math.floor(date.getTime() / 1000 / (24*3600));
            const token = new ethers.Contract(addrs.Token, tokenAbi);
            console.log(BN.from(day))
            const totalBid = await token.connect(provider.getSigner(0)).totalBids(BN.from(day));
            if(totalBid.eq(BN.from(0))) {
                setAmount(0);
            } else {
                token.connect(provider.getSigner(0)).withdrawalAmount(day).then((amount: string) => {
                    setAmount(Number(amount));
                });
            }
        });
    }, [date]);

    async function withdraw() {
        // TODO: Duplicate code
        await (window as any).ethereum.enable();
        const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
        const { chainId } = await provider.getNetwork();
        const addrs = (deployed as any)[CHAINS[chainId]];
        if(!CHAINS[chainId] || !(CHAINS[chainId] in deployed)) {
            alert("This chain is not supported"); // TODO
            return;
        }
        const token = new ethers.Contract(addrs.Token, tokenAbi);
        const day = Math.floor(Number(date) / (24*3600))
        await token.connect(provider.getSigner(0)).withdraw(day, await provider.getSigner(0).getAddress(), {
            gasLimit: '200000',
        });
    }
    
    // TODO: Withdrawal to other account.
    return (
        <>
            <p>Withdraw for bid date:</p>
            <Calendar maxDate={maxDate} defaultValue={maxDate} onChange={setDate}/>
            <p><button onClick={withdraw}>Withdraw</button> <span>{amount === 0 ? '' : utils.formatEther(amount)}</span> CT{" "}
                {withdrawn ? "already withdrawn" : "not withdrawn"}
            </p>
        </>
    );
}
