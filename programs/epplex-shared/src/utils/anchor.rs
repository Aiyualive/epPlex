// Closing a pda that has changed its structure
// https://solana.stackexchange.com/questions/11606/how-to-close-a-pda-account-when-the-structure-has-been-changed
pub fn close<'info>(info: AccountInfo<'info>, sol_destination: AccountInfo<'info>) -> Result<()> {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;

    info.assign(&system_program::ID);
    info.realloc(0, false).map_err(Into::into)
    // info.realloc(0, false).map_err(Into::<anchor_lang::prelude::ProgramError>::into);
}
