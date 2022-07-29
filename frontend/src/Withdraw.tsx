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
    const [userAccount, setUserAccount] = useState<string | null>();

    function handleAccountsChanged(accounts: any) {
        if(accounts[0]) {
            setUserAccount(accounts[0]);
        }
    }

    // TODO: duplicate code
    (window as any).ethereum
        .request({ method: 'eth_accounts' })
        .then(handleAccountsChanged)
        .catch((err: any) => {
            console.error(err);
        });
    (window as any).ethereum.on('accountsChanged', handleAccountsChanged);

    useEffect(() => {
        async function doIt() {
            // TODO: Duplicate code
            if(userAccount !== null) {
                const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
                const { chainId } = await provider.getNetwork();
                const addrs = (deployed as any)[CHAINS[chainId]];
                const day = Math.floor(date.getTime() / 1000 / (24*3600));
                const token = new ethers.Contract(addrs.Token, tokenAbi, provider.getSigner(0));
                const totalBid = await token.totalBids(BN.from(day));
                if(totalBid.eq(BN.from(0))) {
                    setAmount(0);
                } else {
                    token.withdrawalAmount(day, userAccount)
                        .then((amount: string) => {
                            setAmount(Number(amount));
                        });
                }
            }
        }
        doIt()
    }, [date, userAccount]);

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
        await token.connect(provider.getSigner(0)).withdraw(day, userAccount, {
            gasLimit: '200000',
        });
    }
    
    // TODO: Withdrawal to other account.
    return (
        <>
            <p>Withdraw for bid date:</p>
            {/*<Calendar maxDate={maxDate} defaultValue={maxDate} onChange={setDate}/>*/}
            <Calendar defaultValue={maxDate} onChange={setDate}/>
            <p><button onClick={withdraw}>Withdraw</button> <span>{amount === 0 ? '' : utils.formatEther(amount)}</span> CT{" "}
                {withdrawn ? "already withdrawn" : "not withdrawn"}
            </p>
        </>
    );
}
