local function is_bsd(conn)
	local re = regex.compile("NetBSD")
	local cmd_out = conn:run_cmd("uname -a")

	if re:is_match(cmd_out) then
		return "PASS", "SSH configuration is secure."
	else
		return "FAIL", "Not NetBSD"
	end
end

-------------------------------------------------------------------
-- THE REGISTRATION PHASE
-- This is where the script talks to your Rust host.
-------------------------------------------------------------------

register_check({
	id = "BSD-123",
	name = "Is NetBSD",
	description = "Checks if the target is NetBSD",
	severity = "Info",
	run = is_bsd,
})
