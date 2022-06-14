// import { ethers } from 'hardhat';
import { ethers } from 'ethers';
import Calendar from 'react-calendar'
import 'react-calendar/dist/Calendar.css';
import deployed from "../../dist/deployed-addresses.json";
import { CHAINS } from './data';
import { abi as tokenAbi } from "../../artifacts/contracts/Token.sol/Token.json";
import { useEffect, useState } from 'react';
const { utils, BigNumber: BN } = ethers;

export default function Withdraw() {
    const maxDate = new Date();
    // maxDate.setDate(maxDate.getDate() - 1);
    const [date, setDate] = useState(maxDate);
    const [amount, setAmount] = useState<number | null>(null);
    const [withdrawn, setWithdrawn] = useState(false);

    useEffect(() => {
        // TODO: Duplicate code
        (window as any).ethereum.enable().then(async () => {
            const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
            const { chainId } = await provider.getNetwork();
            const day = Math.floor(date.getTime() / 1000 / (24*3600));
            const token = new ethers.Contract(deployed[CHAINS[chainId]].Token, tokenAbi);
            const totalBid = await token.connect(provider.getSigner(0)).totalBids(BN.from(day));
            if(totalBid.eq(BN.from(0))) {
                setAmount(0);
            } else {
                token.connect(provider.getSigner(0)).withdrawalAmount(day).then(amount => {
                    setAmount(amount);
                });
            }
        });
    }, [date]);

    async function withdraw() {
        // TODO: Duplicate code
        await (window as any).ethereum.enable();
        const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
        const { chainId } = await provider.getNetwork();
        if(!CHAINS[chainId] || !deployed[CHAINS[chainId]]) {
            alert("This chain is not supported"); // TODO
            return;
        }
        const token = new ethers.Contract(deployed[CHAINS[chainId]].Token, tokenAbi);
        const day = Math.floor(Number(date) / (24*3600))
        await token.connect(provider.getSigner(0)).withdraw(day, await provider.getSigner(0).getAddress(), {
            gasLimit: '200000',
        });
    }
    
    // TODO: Withdrawal to other account.
    return (
        <>
            <p>Withdraw for bid date: <Calendar maxDate={maxDate} onChange={setDate}/></p>
            <p><button onClick={withdraw}>Withdraw</button> <span>{amount === null ? '' : utils.formatEther(amount)}</span> WT{" "}
                {withdrawn ? "already withdrawn" : "not withdrawn"}
            </p>
        </>
    );
}
