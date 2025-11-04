function is_bsd(ssh_session)
	local re = regex.compile("NetBSD")
	local command_output = ssh_session:run_cmd("uname -a")
	if re:is_match(command_output) then
		return true, "All good"
	else
		return false, "No good"
	end
end
