import React from "react";
import { useEffect, useState } from "react";
import validators from "./validators";

// TODO: ENS
export function EthAddress(props: {
    onValid?: (valid: boolean) => void,
    onChange?: (value: string) => void,
    value?: string,
    defaultValue?: string
}) {
    // const [value, setValue] = useState(props.defaultValue || undefined);
    const [valid, setValid] = useState(false);

    useEffect(() => {
        const valid_ = props.value !== undefined && validators.isEthAddressValid(props.value);
        setValid(valid_);
        if(props.onValid) {
            props.onValid(valid_);
        }
    }, [props.value]);

    // useEffect(() => {
    //     setValue(props.value || props.defaultValue);
    // }, [props.value])

    return <input type="text" defaultValue={props.defaultValue} value={props.value} className={valid ? "" : "error"}
        onChange={(e: any) => { if(props.onChange) { props.onChange(e); }}} />
}